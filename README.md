# Shecret

![](./images/sean.jpg)

## 1. Description

Shecret is a CLI SSH/SFTP client

## 2. Features

* Store server connections on SQL
* Send server connection command to clipboard (control + v)
* Create SSH keys (ed25519 algorithm)
* Send SSH command to multiple servers at once (sort of what [Ansible](https://www.ansible.com/) does)
* Check if servers are online/offline (ping)


## 3. Demo

![](./images/demo.gif)


## 4. Instalation
### 4.1 Cargo

    cargo install shecret

### 4.2 Ready-to-use executable

|OS|Architecture| File*|
|--|--|--|
|Linux|x86_64|[shecret](https://github.com/costa86/shecret/blob/master/shecret)|

*Make sure you've granted executable permissions to it

    ./shecret

## 5. Documentation

https://docs.rs/shecret/latest/shecret/