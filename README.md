# Logger client
Server repo: <https://github.com/seungjin/logger>

## Set LOGGER_AUTHKEY env variable 
```consle
export LOGGER_AUTHKEY=YOUR_KEY_HERE
```

## Run
```console
Logger client

Usage: logger-client <--sock <SOCK>|--pipe <PIPE>>

Options:
  -s, --sock <SOCK>  
  -p, --pipe <PIPE>  
  -h, --help         Print help
  -V, --version      Print version
```

Run with Socket
```console
# logger-client --sock foo
Socket interface selected
Local socket path: /run/user/1000/seungjin-logger/foo
Remote log endpoint: https://LOG.SERVER/SENDER_HOSTNAME/foo
...
User quites: <Ctrl+C> received.
Socket removed: /run/user/1000/seungjin-logger/foo
```

Run with Pipe
```console
# echo "hello" | logger-client --pipe foo
Remote log endpoint: https://LOG.SERVER/SENDER_HOSTNAME/foo
```


