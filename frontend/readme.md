

## run locally 
cd frontend 
trunk server

## run; make accissble in local network

### frontend
```
API_URL=<backend url>:3000 trunk serve --address 0.0.0.0
```
sample 
```
API_URL=http://192.168.178.117:3000 trunk serve --address 0.0.0.0
```

### backend 
```
cargo run server --host 0.0.0.0
```
