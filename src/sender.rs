use crate::config;
use ipp::{attribute::*, client::*, prelude::*};
use std::fs::File;

pub struct Sender {
    printer: config::Printer,
    style: config::Style,
    client: IppClient,
    uri: Uri,
    attrs: Vec<IppAttribute>,
}

impl Sender {
    pub fn new(
        printer: config::Printer,
        style: config::Style,
    ) -> Result<Sender, Box<dyn std::error::Error>> {
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

        Ok(Sender {
            printer,
            style,
            client,
            uri,
            attrs,
        })
    }

    // take a vec of words, format it, then send to printer
    pub async fn print_zpl_with_message(
        &self,
        message: Vec<String>,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let font_size = self.style.font_size;

        let mut fo_acc = 40;
        let mut label_body = String::new();
        // Create formatting for each individual line
        for line in message {
            label_body = format!("{}^FO10,{}^FD{}^FS", label_body, fo_acc, line);
            fo_acc += font_size + self.style.line_padding;
        }

        let mut invert = "N";
        if self.style.invert {
            invert = "I"
        }

        let zpl = format!(
            "^XA^CF{},{}^PO{}{}^XZ",
            self.style.font, font_size, invert, label_body
        );
        self.print_zpl_string(zpl).await
    }

    /// Get the attributes associated with the printer
    pub async fn get_attrs(&self) -> Result<IppAttributes, Box<dyn std::error::Error>> {
        let operation = IppOperationBuilder::get_printer_attributes(self.uri.clone()).build();
        let resp = self.client.send(operation).await?;
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

    /// Print from a ZPL file located at zpl_path
    pub async fn print_zpl_file(
        &self,
        zpl_path: String,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let zpl_file = File::open(zpl_path)?;
        let payload = ipp::payload::IppPayload::new(zpl_file);
        self.print_zpl_payload(payload).await
    }

    /// print from a specified ZPL string
    pub async fn print_zpl_string(&self, zpl: String) -> Result<i32, Box<dyn std::error::Error>> {
        let bytes = ZplReader::new(zpl);
        let reader = std::io::BufReader::new(bytes);
        let payload = ipp::payload::IppPayload::new(reader);
        self.print_zpl_payload(payload).await
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
