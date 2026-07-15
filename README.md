# Simple Sleep
_System sleep and hibernate utility as a PAM service, written in Rust._

### Rationale

> There exist plenty of ways to put your machine to sleep, so why make another one?

The answer is simple! I don't like using systemd, and I take that one step beyond using openrc. I additionally avoid using even elogind on my personal machines. This has the caveat of making rootless suspend operations a bit difficult, such as configuring a window manager to put the device to sleep on a laptop lid switch event.

Around the same time I decided to try my luck at moving away from elogind, I learned about the role of PAM, and quickly I realized the potential it had! I also thought this might be a good place to get my Rust toes into the water.

### Usage

This sleep utility works by taking advantage of the SUID bit provided as a feature of Linux for processes that need to run at elevated permissions. In order to authenticate the calling user, the program invokes the PAM service named "power-state" (located in `/etc/pam.d/power-state`.) The default configuration I've provided checks if the calling user is the root user _OR_ if the user belongs to the `power` group. If so, authentication succeeds, and the Pprogram will perform the respective sysfs write.

Issuing a sleep is simple, just invoke the process:
```bash
power-state
```

Optionally, one can provide the suspend type:
```bash
power-state sleep
power-state hibernate
```

### Caveats

As with many of my projects, I only get them about 80% baked.

- There almost certainly exists plenty of additional functionality that could be added, much of which around the system suspend logic sits well outside my current scope of knowledge. This program is not a novel new way to suspend the system, it's just a wrapper for the existing Linux API. (Thus, this requires a kernel versioned at least 4.X.X ....roughly.)

- Another such missing functionality is any check for the supported sleep types. Right now we just bubble up the error, if one occurs.

- The use case for this project is quite niche. If you _do_ have elogind on your system (you probably do,) then this program likely offers you no benefit. Just call:
```bash
loginctl suspend
```

### Building / Installation

**The build script is currently a WIP!**

But a manual installation is pretty straightforward:
1) Clone this repo
2) In the repo root, run `cargo build`
3) On the output binary, perform the following commands:  
    3a) `chown root:root power-state` (requires elevation)  
    3b) `chmod 4755 power-state` (requires elevation)  
4) Place the binary in your binary location of choice, I recommend `/usr/local/bin`
`cp power-state /usr/local/bin/power-state` (requires elevation, probably)
5) Copy the pam service to the respective directory
`cp pam.d/power-state /etc/pam.d/power-state`
6) Ensure that your user is authenticated, perform the following:  
    6a) `groupadd power` (if the group does not already exist, requires elevation)  
    6b) `usermod -aG power <your user>` (requires elevation)  

#### Dependencies

**This is also a WIP! I still need to iron out exactly what all is required to build this. YMMV!**

- Rust / Cargo (I know fedora is weird and also needs `cargo-devel`)
- LLVM (sometimes included with the above, maybe not always though; alpine wanted `llvm-dev`)
- Clang (Specifically something that provides libclang.so; alpine wanted `clang-libclang`)
- PAM (many distros also probably need something like a `pam-devel`)

### Contributing

I love receiving PRs! If you have an idea please feel free to submit an issue or a PR. I promise I will look at all of them in time. (Unfortunately my day-to-day is packed enough that I am shamefully slow to reply at times. Rest assured, your contribution is either accepted, rejected, or I have not yet had a chance to review it.)

If you do submit a PR, please consider the following guidelines:
- Prefer isolating changes to their own PRs -- I'd prefer multiple back-to-back pull requests instead of one big one, if possible! This makes it easier to granularly merge some features while requesting revision on others!
- Please disclose AI-generated code! (Same for AI-discovered issues, please!) -- This will not invalidate your pull request, but it helps me to better select the correct approach for code review.
