# Zebrasend

`zebrasend` is a small rust utility for sending commands and print jobs to Zebra label printers via CUPS, SGD, and ZPL.

## Config

`zebrasend` uses a small config file to set the printer CUPS/IPP URL, as well as style and print settings. See `zebrasend.toml` for a maximal example.

## Examples

```
# Send a ZPL file to a printer
zebrasend file home_address.zpl

# Send a text message to the printer. Each word will be printed on a new line, unless surrounded by quotes.
zebrasend message "my favorite printer is the" GX430t

# Send a SGD command to the printer
zebrasend sgd get memory.ram_size
"2054496 Bytes"
```