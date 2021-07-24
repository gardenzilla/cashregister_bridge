# Printer bridge

Bridge between local default printer and websocket.

You can send a pdf file as base64 encoded text to the opened websocket, 
the bridge is going to decode it and foreward to the default printer.

## Using

you are going to mainly use the followings:

```shell
make dev
```

```shell
make release
```