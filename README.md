# Zebrasend

`zebrasend` is a small rust utility for sending commands and print jobs to Zebra label printers via SGD, FTP jetdirect and ZPL.

## Config

`zebrasend` uses a small config file to set the printer URLs and style settings, as well as style and print settings. See `zebrasend.toml` for a maximal example.

## Usage

```
zebrasend --help
zebrasend 0.1.0
A CLI utility for sending commands to zebra printers via CUPS

USAGE:
    zebrasend [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -c, --config-file <FILE>    
    -h, --help                  Print help information
    -p, --printer <PRINTER>     Printer from the specified config to use [default: default]
    -s, --style <STYLE>         Style from the specified config to use [default: default]
    -V, --version               Print version information

SUBCOMMANDS:
    file        Send a ZPL file to the printer
    ftp         send and delete files via FTP
    help        Print this message or the help of the given subcommand(s)
    message     Send a string message to the printer. Individual words will be printed to new
                    lines, use quotes to print on a single line
    printers    Print a list of configured printers
    sgd         Send SGD commands to printer via telnet
    styles      Print a list of configured styles
```

```
# Send a ZPL or NRD file to a printer
zebrasend file home_address.zpl

# Send a ZPL file via FTP
zebrasend ftp put home_address.zpl

# Send a text message to the printer. Each word will be printed on a new line, unless surrounded by quotes.
zebrasend message "my favorite printer is the" GX430t

# Send a SGD command to the printer
zebrasend sgd get memory.ram_size
"2054496 Bytes"

```

## Example ZPL files
The `zpl_files` folder contains a number of example ZPL files, including a variety of barcodes.
Note that most of these files were developed and tested on a 300DPI printer with 3"x3" paper.