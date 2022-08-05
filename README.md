# Zebrasend

`zebrasend` is a small rust utility for sending commands and print jobs to Zebra label printers via CUPS, SGD, jetdirect and ZPL.

## Config

`zebrasend` uses a small config file to set the printer CUPS/IPP URL, as well as style and print settings. See `zebrasend.toml` for a maximal example.

## Examples

```
zebrasend 0.1.0
A CLI utility for sending commands to zebra printers via CUPS

USAGE:
    zebrasend [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -c, --config-file <FILE>         
    -h, --help                       Print help information
    -m, --print-mode <PRINT_MODE>    print mode to use for file and message subcommands [default:
                                     cups] [possible values: jetdirect, cups]
    -p, --printer <PRINTER>          Printer from the specified config to use [default: default]
    -s, --style <STYLE>              Style from the specified config to use [default: default]
    -V, --version                    Print version information

SUBCOMMANDS:
    file       Send a ZPL file to the printer
    help       Print this message or the help of the given subcommand(s)
    message    Send a string message to the printer. Individual words will be printed to new
                   lines, use quotes to print on a single line
    options    Return the options known to the printer
    sgd        Send SGD commands to printer via telnet
```

```
# Send a ZPL file to a printer
zebrasend file home_address.zpl

# Send a text message to the printer. Each word will be printed on a new line, unless surrounded by quotes.
zebrasend message "my favorite printer is the" GX430t

# Send a SGD command to the printer
zebrasend sgd get memory.ram_size
"2054496 Bytes"

# If properly configured, print commands can be sent over either jetdirect or cups
zebrasend -m jetdirect message "my favorite printer is the" GX430t
```