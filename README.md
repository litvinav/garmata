<p align="center">
<img src="./docs/img/logo.png" alt="logo" width="380"/>
</p>
🔫 Garmata (Cyrillic: гармата, [гарма́та], weapon in Belarusian) is a self contained and minimalistic performance testing tool to put a system under load.

This project is work-in-progress and may introduce big and breaking changes between version. Consult this and other docs for the usage of the current version.

# How to install

Either get a prebuild version from the Release page or build from source files.

# Usage
Usage of the executable:
```sh
garmata ./path/to/configuration.yaml # assuming ./configuration.yaml if no path provided
```

Example configuration file:
```yaml
# scheme: http # default https
target: httpbin.org # host; port is assumed based on scheme is not provided
# http_version: "1.1" # default version is 2
playlist:
- name: Get Group # optional name for stats group
  duration: 10 # duration in seconds
  flow:
  - name: Request page # optional name for stats group
    path: /get
    method: GET
- name: Post Group
  duration: 10
  flow:
  - name: Post to target
    path: /post
    method: POST
```
All plays run in parallel. Each play executes the flow steps sequentially and repeats until the duration deadline.

Example output:
```
Playlist: Get Group
  Flow: Request page
    min: ......................................................................... 0.5371544s
    avg: ......................................................................... 0.6625912s
    p50: ......................................................................... 0.66122407s
    p95: ......................................................................... 0.74812794s
    max: ......................................................................... 0.76736563s
Playlist: Post Group
  Flow: Post to target
    min: ......................................................................... 0.5463552s
    avg: ......................................................................... 0.66272366s
    p50: ......................................................................... 0.6617749s
    p95: ......................................................................... 0.7485342s
    max: ......................................................................... 0.7673268s
```
