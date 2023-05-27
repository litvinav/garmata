<p align="center">
<img src="./docs/img/logo.png" alt="logo" width="380"/>
</p>
🔫 Garmata (Cyrillic: гармата, [ˈɡɑrmɑtɑ], weapon in Belarusian) is a headless, self contained and minimalistic performance testing tool to put a system under load.

> This project is work-in-progress and may introduce big and breaking changes between versions. Minor and Patch changes may rename or restructure parts of this tool. Please consult the help page with the `--help` flag for your executable or the doc pages for the usage of the current version.

# How to install

Either get a prebuild version from the assets in the [Releases page](https://github.com/litvinav/garmata/releases) or build a executable from source files.

<details>
<summary>Help me to setup the executable</summary>
<p>Download the executable fit for your system based on the name and rename it to "garmata" for Unix or "garmata.exe" for Windows.</p>
<p>Only x86_64 versions are present in the Releases page. Thanks to Rosetta you can still use it on your M1 or M2 Mac.</p>
<p>For Unix systems you have to give the binary file executable permissions to make it a executable: "chmod +x ./file-name" 
Either place the binary in "/usr/bin/garmata" or store it user scoped in your home folder "~/bin/garmata".</p>
<p>In case MacOS complains that the executable is not downloaded from a trusted source, the best way to fix this issue (that i am aware of) is to right click the execurable in finder and select "open with ..." and then "iTerm". If you confirm the next popup by clicking "OK", your garmata executable will be excluded from this protection rule. MacOS will keep warning you for other executables not installed from the AppStore.</p>
<p>For Windows you can store the executable in e.g. "C:\Program Files\litvinav\garmata.exe" and you want to make sure the executable is located in one of the directories reachable by the PATH variable. If the location is referenced via PATH, you can execute "garmata.exe --help" in cmd or PowerShell.</p>
<p>Don't forget to add the executable location to the PATH variable either way for personal usage. Of course you can call it directly via the full path to the executable for one-off usage in e.g. Runners for over night load testing.</p>
</details>

# Usage
Currently garmata can output the stats summary, payload's send and received or the full times as csv to stdout.
```sh
# Perform the test configured in "./configuration.yaml" and print the stats as a summary to stdout
garmata
```
```sh
# Perform the test configured in "./configuration.yaml" and print the requests and responses to stdout
garmata -o debug
```
```sh
# Collect performance data for test configured in "./test.yaml" and output as csv into results.csv
garmata -o csv ./test.yaml 1> results.csv
# Analyze the output in the program of your choice.
libreoffice.calc results.csv  
```
Use the `--help` flag to see all usage instructions.

Minimal configuration file:
```yaml
target: example.com
groups:
- flows:
  - path: /
    method: GET
```

All configurable fields:
```yaml
scheme: https # http or https only atm; default is https if not provided
target: httpbin.org # scoped host target; port can be auto resolved based on the request scheme
http_version: "1.1" # default is 1.1 if not provided
groups:
- name: API Backend # A name for stats group. Optional but recommended.
  users: 2 # default is 1 if not provided
  duration: 10 # duration in seconds; default is 0 or "run once" if not provided
  flows:
  - name: Profile edit route # A name for stats flow. Optional but recommended.
    path: /anything
    max_redirects: 10 # max for one iteration; default is 0 if not provided
    method: POST # any http method; uppercased but not validated (check for typos)
    body: '{ "hello": "world" }' # optional
    insecure: false # default is false if not provided; allows insecure/self-signed certificates if true
    headers: # optional
      # Garmata sets 2 common browser headers but override is possible. In case of duplicates the last wins.
      accept: application/json # default is "*/*" as in most browsers
      accept-encoding: "" # default is "gzip, deflate, br" as in most browsers
    cookies: # optional; response set-cookie syntax (for easy copy paste)
    - "theme=dark"
    - "Session=1; Path=/profile"

```
All groups run in parallel times the amount of users. Each group executes the flow steps sequentially and repeats them until the duration deadline.

# Strategy / Roadmap

Garmata's first aim is to become a handy and convenient web testing tool. No cloud, no other hard dependencies, no lock-ins, no payments or subscriptions, no bloat but also not feature rich. 

Tests with Garmata should be easy and fast to set up, as well as reproducable and straight forward.

If possible, the code should be understandable for Open Source analysis by senior and experienced programmers. Understandable code means code and not comments for every line or functions for every action. The current state might not reflect the desired state.

In the future Garmata might be exported as a cargo lib for embeded testing. Also when this library is mature, it might be exported as a http client for general use, but only time will tell.
