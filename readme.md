<br/>
<p align="center">
    <img src="https://raw.githubusercontent.com/clowzed/sero/12c8a570755b2f0c1fb570167ce7dcc4b0c323e9/logo.svg" alt="Logo" width="80" height="80">  

  <h3 align="center">sero</h3>

  <p align="center">
    Lightning-fast, static web publishing with zero configuration and full control
    <br/>
    <br/>
<div align = "center">

![Postgres](https://img.shields.io/badge/postgres-%23316192.svg?style=for-the-badge&logo=postgresql&logoColor=white)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![Nginx](https://img.shields.io/badge/nginx-%23009639.svg?style=for-the-badge&logo=nginx&logoColor=white)

![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)
</div>

  </p>
</p>

## 📖 Table Of Contents
- [📖 Table Of Contents](#-table-of-contents)
- [🔧 Tools](#-tools)
- [❓ About The Project](#-about-the-project)
- [🚀 Features](#-features)
- [🔌 Built With](#-built-with)
- [⌨️ Getting Started](#️-getting-started)
	- [Prerequisites](#prerequisites)
	- [Installation](#installation)
		- [Clone this repository](#clone-this-repository)
		- [✏️ Configure DNS records](#️-configure-dns-records)
		- [✏️ Configure `docker-compose.yml`](#️-configure-docker-composeyml)
		- [🚀 Deploy](#-deploy)
- [✨ Usage](#-usage)
	- [Installing cli tool](#installing-cli-tool)
	- [🆕 Creating and initializing folder](#-creating-and-initializing-folder)
	- [🆕 Creating index.html](#-creating-indexhtml)
	- [🔍 Inspecting `sero.toml`](#-inspecting-serotoml)
	- [✏️ Changing `sero.toml`](#️-changing-serotoml)
	- [⬆️ Registration and uploading](#️-registration-and-uploading)
		- [Now your site is available at http://oursite.awesome.com/index.html](#now-your-site-is-available-at-httpoursiteawesomecomindexhtml)
	- [Advanced usage with `new` features](#advanced-usage-with-new-features)
- [📍 Roadmap](#-roadmap)
- [🧑‍🤝‍🧑 Contributing](#-contributing)
	- [Creating A Pull Request](#creating-a-pull-request)
- [License](#license)
- [Authors](#authors)



## 🔧 Tools
**Sero - [this tool is used to upload your site to sero servers](https://github.com/clowzed/seroup)**



## ❓ About The Project
This project is essentially a revamp of the well-known surge.sh platform. While surge.sh is a fantastic tool for publishing web applications, I noticed it lacked certain features that could significantly enhance its utility. Therefore, I decided to create my own version, incorporating those missing elements to provide a more comprehensive and seamless user experience.

One key feature that it is self-hosted. This gives users more flexibility and control over their projects, allowing them to truly make their work their own. My goal with this project is to create a platform that not only meets but exceeds the needs of web developers, making the process of publishing web applications as hassle-free and efficient as possible. It also has some features that are not in surge.sh

**So saying shortly this is a simple web server for static websites but with an ability to deploy it with custom subdomain without any configuration using cli tool. On upload it will automatically create subdomain for your site.**

**This is a cli tool for upload [tool](https://github.com/clowzed/seroup)**

## 🚀 Features
- Deploy in seconds without configuration
- Enable and disable site `new`
- Download site `new`
- Limits control `new`
- Easy upload with cli tool
- Custom 404.html `(on 404 status user will see your 404.html)`
- Custom 503.html `new` `(on disabled site)`
- Clean urls

## 🔌 Built With
- `Rust`
- `Sea-orm` - [`sero is added to community examples`](https://github.com/SeaQL/sea-orm/blob/06c632712f3d167df0cda742dd228661b953ab7f/COMMUNITY.md?plain=1#L28)
- `Axum`  - [`sero is added to community examples`](https://github.com/tokio-rs/axum/blob/d7258bf009194cf2f242694e673759d1dbf8cfc0/ECOSYSTEM.md?plain=1#L78)
- `Postgres`
- `Nginx`

## ⌨️ Getting Started


### Prerequisites

That is what you will need to deploy this project

* You need to buy a domain
* You need a server
* You need access to DNS records
* You need to have `docker-compose` installed


### Installation

#### Clone this repository
```bash
git clone https://github.com/clowzed/sero
```

#### ✏️ Configure DNS records

- Add wildcard subdomain to your server.
   
It is usually done by adding `TXT` DNS record with value `"*"` pointing to your server

#### ✏️ Configure `docker-compose.yml`

Simply open a [docker-compose.yml](https://github.com/clowzed/sero/blob/master/docker-compose.yml) file from this repo with any redactor you link

1. Configure your domain and zone (for example I bought example.com) (lines 30 and 31)


| environment variable | description |
|----------------------|-------------|
| DOMAIN               | example     |
| ZONE                 | com         |

2. Configure desired limits if you want (you can skip this)

| environment variable | description                                        | already set     |
|----------------------|----------------------------------------------------|-----------------|
| MAX_USERS            | Maximum amount of users to be registered           | 1               |
| MAX_SITES_PER_USER   | Maximum amount of sites which each user can upload | 100             |
| MAX_BODY_LIMIT_BYTES | Maximum body limit in bytes                        | 10000000 (10mb) |

#### 🚀 Deploy

```bash
docker-compose up -d
```

## ✨ Usage

Let see an example of uploading your site to your sero server.

Consider our domain is awesome.com

### Installing cli tool
```bash
npm i -g @clowzed/sero
```

### 🆕 Creating and initializing folder
```bash
mkdir our-website
cd our-website
sero init # This will generate default sero.toml file
```

### 🆕 Creating index.html
```bash
echo "Hello from our website!" > index.html
```

### 🔍 Inspecting `sero.toml`
Tha is how default sero.toml file looks like
```toml
[credentials]
username = "clowzed" # Here will be your hostname
password = ""

[server]
url = "http://sero.com/" 
subdomain = "clowzed"
```

### ✏️ Changing `sero.toml` 
So now we need to change url to point to our server.

We also want to change subdomain for our website
```toml
[credentials]
username = "clowzed"
password = ""

[server]
url = "http://awesome.com/" 
subdomain = "oursite"
```

### ⬆️ Registration and uploading
```bash
sero register # We need to call it this only if we've changed username
sero upload
```

#### Now your site is available at [http://oursite.awesome.com/index.html]()

### Advanced usage with `new` features

1) Disabling site `new`

You can disable your site using this command.
```bash
sero disable
```
This will preserve your subdomain so other users will not be able to borrow it.
This will return `503 status code` for any request to site with your subdomain.
You can create `503.html` file so it will be returned to user. You can do it for maintenance. The `503.html` file should be at root of your folder

2) You can enable site with this command. Now it will work as usual. `new`
```bash 
sero enable
```
3) Download your site `new`

You can easily download your site as zip with this command
```bash
sero download
```

4) Delete your site and free subdomain
```bash
sero teardown
```



## 📍 Roadmap

See the [open issues](https://github.com/clowzed/sero/issues) for a list of proposed features (and known issues).

## 🧑‍🤝‍🧑 Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.
* If you have suggestions for adding or removing projects, feel free to [open an issue](https://github.com/clowzed/sero/issues/new) to discuss it, or directly create a pull request after you edit the *README.md* file with necessary changes.
* Please make sure you check your spelling and grammar.
* Create individual PR for each suggestion.
* Please also read through the [Code Of Conduct](https://github.com/clowzed/sero/blob/main/CODE_OF_CONDUCT.md) before posting your first idea as well.

### Creating A Pull Request

1. Fork the Project
2. Create your feature Branch (`git checkout -b feature/some`)
3. Commit your changes (`git commit -m 'implementation of some feature'`)
4. Push to the branch (`git push origin feature/some`)
5. Open a Pull Request 

## License

[Distributed under the MIT License](https://github.com/clowzed/sero/blob/master/LICENSE)

## Authors

* **clowzed** 

