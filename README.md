# CI dashboard

The user interface for CI jobs status display. 
Currently only for GitLab API.

This project has been started to learn the Rust language and WASM universe. 

## Development

### Requirements / Setup

Install wasm toolchain:

* wasm-pack
    
```
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

* npm, example for Ubuntu: 

```
    sudo apt install npm
```

* ensure npm latest version

```
    npm install npm@latest -g
```
      
See further documentation at:

https://rustwasm.github.io/book/game-of-life/setup.html


### Build

To update rust code, run wasm build from repository root.

```
wasm-pack build
```

Change to pkg folder, link the local package to avoid 
publishing during development:

``` 
npm link
```

Change to www folder, 
install or update npm dependencies and use locally 
linked package:

``` 
npm install
npm link ci-dashboard
```


### Run development webserver

Copy config_sample.json to config.json, edit config.json and enter your API token.


From www folder execute:

``` 
npm run start -- --port 8080
```

Open http://localhost:8080

