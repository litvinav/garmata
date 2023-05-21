<p align="center">
<img src="./docs/img/logo.png" alt="logo" width="380"/>
</p>
🔫 Garmata (Cyrillic: гармата, [гарма́та], weapon in Belarusian) is a self contained and minimalistic performance testing tool to put a system under load.

> This project is work-in-progress and may introduce big and breaking changes between versions. Minor and Patch changes may rename or restructure parts of this tool. Please consult the help page with the `--help` flag for your executable or the doc pages for the usage of the current version.

# How to install

Either get a prebuild version from the assets in the [Releases page](https://github.com/litvinav/garmata/releases) or build a executable from source files.

# Usage

Run garmata with the `--help` flag to see the usage instructions.
```sh
garmata -o csv ./test.yaml 1> results.csv
```
Currently garmata can output the stats summary or the full times as csv to stdout.

Example configuration file:
```yaml
scheme: https # http or https only atm; default is https if not provided
target: example.com # scoped host target; port can be auto resolved based on the request scheme
http_version: "1.1" # default is 1.1 if not provided
groups:
- name: Commmon access # optional name for stats group
  duration: 10 # duration in seconds
  flow:
  - name: landing page # optional name for stats group
    path: /
    method: GET
    headers: # optional hashmap of additional request headers
      Proxy-Authentication: Bearer rIObAeA6W4ysAUDzTJAz9DzvLBGGO60T
```
All groups run in parallel. Each group executes the flow steps sequentially and repeats them until the duration deadline.
