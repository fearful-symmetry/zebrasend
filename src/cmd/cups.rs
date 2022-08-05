use crate::config;
use ipp::{attribute::*, client::*, prelude::*};
use std::fs::File;
use tokio::runtime::Runtime;

pub struct Cups {
    printer: config::Printer,
    client: IppClient,
    uri: Uri,
    attrs: Vec<IppAttribute>,
}

/// Sender is a CUPS wrapper for sending ZPL jobs to the print server
impl Cups {
    pub fn new(printer: config::Printer) -> Result<Cups, Box<dyn std::error::Error>> {
        let uri: Uri = printer.uri.parse()?;
        let client = IppClient::new(uri.clone());

        // construct the basic attributes needed for all printing jobs
        let attrs = vec![
            IppAttribute::new(
                IppAttribute::PRINTER_URI,
                IppValue::Keyword(printer.uri.clone()),
            ),
            IppAttribute::new(
                "zePrintMode",
                IppValue::Keyword(printer.ze_print_mode.clone()),
            ),
            IppAttribute::new(
                "document-format",
                IppValue::MimeMediaType("application/vnd.cups-raw".to_string()),
            ),
            IppAttribute::new(
                IppAttribute::REQUESTING_USER_NAME,
                IppValue::NameWithoutLanguage(printer.user_name.clone()),
            ),
        ];

        Ok(Cups {
            printer,
            client,
            uri,
            attrs,
        })
    }

    /// Get the attributes associated with the printer
    pub fn get_attrs(&self) -> Result<IppAttributes, Box<dyn std::error::Error>> {
        let operation = IppOperationBuilder::get_printer_attributes(self.uri.clone()).build();
        let rt = Runtime::new()?;
        let resp = rt.block_on(self.client.send(operation))?;
        if resp.header().status_code().is_success() {
            return Ok(resp.attributes().clone());
        }
        let err_string: Box<dyn std::error::Error> = format!(
            "Print-Job failed with error {:?}",
            resp.header().status_code()
        )
        .into();
        Err(err_string)
    }

    /// send a given payload to the printer
    async fn print_zpl_payload(
        &self,
        zpl: ipp::payload::IppPayload,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let print_job_op = IppOperationBuilder::print_job(self.uri.clone(), zpl)
            .attributes(self.attrs.clone())
            .job_title("home_addr.zpl")
            .user_name(self.printer.user_name.clone())
            .build();
        let doc_resp = self.client.send(print_job_op).await?;
        if !doc_resp.header().status_code().is_success() {
            let err_string: Box<dyn std::error::Error> = format!(
                "Print-Job failed with error {:?}",
                doc_resp.header().status_code()
            )
            .into();
            return Err(err_string);
        }
        Ok(fetch_job_id(doc_resp.attributes()).map_or(0, |v| *v))
    }

    /// Print from a ZPL file located at zpl_path
    pub fn send_file(&self, path: String) -> Result<String, Box<dyn std::error::Error>> {
        let zpl_file = File::open(path)?;
        let payload = ipp::payload::IppPayload::new(zpl_file);
        //self.print_zpl_payload(payload).await?;
        let rt = Runtime::new()?;
        let job_id = rt.block_on(self.print_zpl_payload(payload))?;
        Ok(format!("{}", job_id))
    }
    /// print from a specified ZPL string
    pub fn send_string(&self, data: String) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = ZplReader::new(data);
        let reader = std::io::BufReader::new(bytes);
        let payload = ipp::payload::IppPayload::new(reader);
        let rt = Runtime::new()?;
        let job_id = rt.block_on(self.print_zpl_payload(payload))?;
        Ok(format!("{}", job_id))
    }
}

fn fetch_job_id(printer_attrs: &IppAttributes) -> Option<&i32> {
    let job_attrs = printer_attrs
        .groups_of(ipp::model::DelimiterTag::JobAttributes)
        .collect::<Vec<&IppAttributeGroup>>()[0]
        .attributes();
    match job_attrs.get("job-id") {
        Some(id) => id.value().as_integer(),
        None => None,
    }
}

// that IppPayload REALLY doesn't want to take a string
struct ZplReader {
    inner: std::vec::IntoIter<u8>,
}

impl ZplReader {
    fn new(data: String) -> Self {
        ZplReader {
            inner: data.into_bytes().into_iter(),
        }
    }
}

impl std::io::Read for ZplReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for i in 0..buf.len() {
            if let Some(x) = self.inner.next() {
                buf[i] = x;
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}
