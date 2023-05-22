<p align="center">
<img src="./docs/img/logo.png" alt="logo" width="380"/>
</p>
🔫 Garmata (Cyrillic: гармата, [ˈɡɑrmɑtɑ], weapon in Belarusian) is a self contained and minimalistic performance testing tool to put a system under load.

> This project is work-in-progress and may introduce big and breaking changes between versions. Minor and Patch changes may rename or restructure parts of this tool. Please consult the help page with the `--help` flag for your executable or the doc pages for the usage of the current version.

# How to install

Either get a prebuild version from the assets in the [Releases page](https://github.com/litvinav/garmata/releases) or build a executable from source files.

<details>
<summary>Help me to setup the executable</summary>
Download the executable fit for your system based on the name and rename it to garmata for Unix or garmata.exe for Windows.
Only x86_64 versions are present in the Releases page. Thanks to Rosetta you can still use it on your M1 or M2 Mac.

For Unix systems you have to give the binary file executable permissions to make it a executable: `chmod +x ./x86_64-release-name` 
Either place the binary in `/usr/bin/garmata` or store it user scoped in your home folder `~/bin/garmata`.

For Windows you can store the executable in e.g. `C:\Program Files\litvinav\garmata.exe` and you want to make sure the executable is located in one of the directories reachable by the PATH variable. If the location is referenced via PATH, you can execute `garmata.exe --help` in cmd or PowerShell.

Don't forget to add the executable location to the PATH variable either way. Of course you can call it directly via the full path to the executable for one-off usage.
</details>

# Usage

Run garmata with the `--help` flag to see the usage instructions.
Currently garmata can output the stats summary or the full times as csv to stdout.
```sh
# Perform the test configured in "./configuration.yaml" and print the stats as a summary to stdout
garmata
```
```sh
# Collect performance data for test configured in "./test.yaml" and output as csv into results.csv
garmata -o csv ./test.yaml 1> results.csv
# Analyze the output in the program of your choice.
libreoffice.calc results.csv  
```

Minimal configuration file:
```yaml
target: example.com
groups:
- duration: 10
  flow:
  - path: /
    method: GET
```
Example configuration file:
```yaml
scheme: https # http or https only atm; default is https if not provided
target: httpbin.org # scoped host target; port can be auto resolved based on the request scheme
http_version: "1.1" # default is 1.1 if not provided
groups:
- name: API Backend # optional name for stats group
  users: 2 # default is 1 if not provided
  max_redirects: 1 # default is 50 if not provided
  duration: 10 # duration in seconds
  flow:
  - name: Profile edit route # optional name for stats group
    path: /anything
    method: POST
    body: '{ "hello": "world" }'
    headers:
      Content-Type: application/json
      Proxy-Authentication: Bearer rIObAeA6W4ysAUDzTJAz9DzvLBGGO60T

```
All groups run in parallel times the amount of users. Each group executes the flow steps sequentially and repeats them until the duration deadline.
