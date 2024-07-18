# cmux

This lib allows you to parse and generate GSM 07.10 Frames.

## Usage

<!-- USAGE_START -->
```plainstext
A library for parsing GSM 07.10 Frame

Usage: cmux <COMMAND>

Commands:
  generate  Generate GSM 07.10 Frame by given address, control and content field [aliases: g]
  parse     Parse a byte array to GSM 07.10 Frame [aliases: p]
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
<!-- USAGE_END -->

### Generate

<!-- USAGE_GEN_START -->
```plainstext
Generate GSM 07.10 Frame by given address, control and content field

Usage: cmux generate [OPTIONS] <CONTENT>

Arguments:
  <CONTENT>  content field

Options:
  -a, --address <ADDRESS>  address field [default: 7]
  -c, --control <CONTROL>  control field [default: EF]
  -h, --help               Print help
```
<!-- USAGE_GEN_END -->

### Parse

<!-- USAGE_PAR_START -->
```plainstext
Parse a byte array to GSM 07.10 Frame

Usage: cmux parse [HEXSTRING]

Arguments:
  [HEXSTRING]  Bytes array like string. Example: "F9010203F9 F9010203F9"

Options:
  -h, --help  Print help
```
<!-- USAGE_PAR_END -->

## References

- [n_gsm kernel module](https://docs.kernel.org/driver-api/tty/n_gsm.html)
- [GSM 07.10 multiplexing protocol](https://www.3gpp.org/ftp/Specs/archive/07_series/07.10/0710-720.zip)