<div align="center">
    <img src="./logo.svg" style="height:100px">
    <h1> Sero Server </h1>
    <h3>Sero is a web server that allows you to easily host your static sites without pain<br>
The idea was inspired by <a href="https://surge.sh">surge.sh</a> but gives you <b>full control</b>.</h3><br>

![Postgres](https://img.shields.io/badge/postgres-%23316192.svg?style=for-the-badge&logo=postgresql&logoColor=white)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![Nginx](https://img.shields.io/badge/nginx-%23009639.svg?style=for-the-badge&logo=nginx&logoColor=white)

![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)

</div>

## Upload

[Use this tool for uploading your site to sero server in one command](https://github.com/clowzed/seroup)

## Features

- Deploy the site with a custom subdomain in seconds!
- Download the site that you already host!
- Teardown sites that you do not like anymore
- Control amount of users and sites
- Custom 404.html
- Custom 503.html
- Enable and disable site for maintenance

## Installation and deployment

### Requirements

- You need to have access to the DNS configuration of the server

## Step 0:
```
git clone https://github.com/clowzed/sero
```

## Step 1: Configure DNS records

- Add wildcard (\*) subdomain to your server

## Step 2: Configure docker-compose.yml

1. Configure your domain and zone (for example for example.com)

   | environment variable | description |
   |----------------------|-------------|
   | DOMAIN               | example     |
   | ZONE                 | com         |

2. Configure limits

   | environment variable | description                                        | already setted  |
   |----------------------|----------------------------------------------------|-----------------|
   | MAX_USERS            | Maximum amount of users to be registered           | 1               |
   | MAX_SITES_PER_USER   | Maximum amount of sites which each user can upload | 100             |
   | MAX_BODY_LIMIT_BYTES | Maximum body limit in bytes                        | 10000000 (10mb) |

#### Step 2: Deploy

```bash

docker-compose up -d
```

## TODO:

- [ ] UI
- [ ] CORS

# Author

- [@clowzed](https://github.com/clowzed)

# License

- MIT

<style>
*{
font-family: PT Mono, SF Mono, "Courier New"
}
</style>
