Rust Sega Emulator
==================

Conversion of a C emulator, that I'd converted to C++, that I'd converted to
Python, that I've converted to Rust.

Original implementation was based on sega master system technical information from:

        SEGA MASTER SYSTEM TECHNICAL INFORMATION, by Richard Talbot-Watkins, 10th June 1998

        Z80 Instructions
        https://www.zilog.com/docs/z80/um0080.pdf

Building/Running
    Install Rust:
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  
        
        linux (rust toolchain dependencies):
          apt-get install build-essential

    Install SDL:
        linux (debian based): 
                apt-get install libsdl2-dev
        rasbian (64-bit): 
                apt-get install libsdl2-dev
        rasberry pi (ubuntu mate 64-bit): 
                # Release 22.04 LTS (Jammy Jellyfish) 64-bit
                # Need to upgrade so 'sdl2' will install.
                apt-get update
                apt-get upgrade
                apt-get install git curl libsdl2-dev

                # 'pipewire' appears to be a good sound driver on the raspberry pi
                # SDL_AUDIODRIVER=pipewire 
        OSX: 
                brew install sdl2

	Windows Build (from linux):
                sudo apt-get install gcc-mingw-w64
                rustup target add x86_64-pc-windows-gnu
                cargo build --target x86_64-pc-windows-gnu --release

                # For 'sdl' (eg: if getting 'cannot find -lSDL2: No such file or directory')
                sudo apt-get install libsdl2-dev -y
                curl -s https://www.libsdl.org/release/SDL2-devel-2.0.22-mingw.tar.gz | tar xvz -C /tmp
                cp -r /tmp/SDL2-2.0.22/x86_64-w64-mingw32/lib/* ~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib/

                # For raspberry pi.  Or generally, set destination directory to: $(rustup show active-toolchain | sed 's/ .*//')
                cp -r /tmp/SDL2-2.0.22/x86_64-w64-mingw32/lib/* ~/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib/

        Webassembly
                From: https://puddleofcode.com/story/definitive-guide-to-rust-sdl2-and-emscriptem
                Then: https://users.rust-lang.org/t/sdl2-emscripten-asmjs-and-invalid-renderer-panic/66567
                Taken from: https://github.com/therocode/rust_emscripten_main_loop

                sudo apt-get install emscripten
                rustup target add wasm32-unknown-emscripten

                # Your experience may vary, adding explicit handling of 'EM_CONFIG'

                EM_CONFIG=$HOME/.emscripten emcc --generate-config
                  Note: May have to manuall update/adjust available system versions in the '.emscripten' config file

                EM_CONFIG=~/.emscripten cargo build-emscripten
                  Note: It's just an alias for 'cargo build --release --config projects/emscripten/'

                # Start a web server and load in browser (point it to the location reported by the python server)
                python3 -m http.server

                # Drag a rom into the 'rom drop' location in the browser.

                # Note, the configuration file in 'projects/emscripten' are the same as running:
                export EMCC_CFLAGS="-s USE_SDL=2"
                cargo build --target wasm32-unknown-emscripten



                # To try different 'emscripten' sdk versions
                git clone https://github.com/emscripten-core/emsdk.git 
                cd rustsega
                cd ../emsdk
                VER=3.1.23 && emsdk install $VER && emsdk activate $VER && . emsdk_env.sh && cd - && rm -r target && cargo build --release --config projects/emscripten/.cargo/config.toml; python -m http.server && cd -


    Build and run:
        cargo run --release <rom_file>

    Usage: rustsega <cartridge_name> [-d] [-n] [-s <stop-clock>] [-f] [-l]
    
    Rusty Sega Emulator.
    
    Positional Arguments:
      cartridge_name    name of cartridge to run
    
    Options:
      -d, --debug       print PC State Debug Info
      -n, --no-delay    run the emulator with no delay (rather than real-time)
      -s, --stop-clock  number of clock cycles to stop the emulator (for
                        benchmarking)
      -f, --fullscreen  run the emulator in full screen mode.
      -l, --list-drivers
                        list SDL drivers
      --help            display usage information

(Current) Inputs:
    Key mappings (Joystick 1):
    Up: Up, Down: Down, Left: Left, Right: Right
    Fire A: Z, Fire B: X
    Reset: R

    Quit: Escape

Note: Currently 'Quit' doesn't appear to work on Rasbian if audio output is set to HMI, when headphones are connected to the AV Jack (it just hangs).

Dependencies:
   Argument parsing dependency added via:
       cargo add clap --features derive

TODO:

 Non-functional:
    Improve structure (current structure is shortest path to get things running).
    Fix status flag calculations,  cross check with good known Z80 results.

    Clean up 'sega.rs' there's a bit too much 'glue' going on there, that should be shifted out to the submodules.

  Update vdp/cycle comparisons so they support clock rollover (currently just set cycles to 64-bit, but I doubt that's how the master system did it).

 Optimise:
 - Don't do full VDP processing per byte update (check for changes, isolate).
 - Look for CPU cycles
 - Profile

Add more tests
  - Capture Timing, put in fairly full set of op code checks, so op codes can be tidied up (there's currently a lot of repetition).

Sound
  - Set a better/dynamic audio queue length (based on speed/current buffer size, for better sound.)
  - Fix noise/periodic channel. When 'periodic' mode is enabled, it sounds
    worse.  Unsure what 'correct' sounds like (but superficially seems like it
    should have more noise, rather than high pitch pings). The 'noise' sounds
    reasonable, but not sure how accurate it is (currently have a frequency
    multiplier that probably isn't correct).

  
Constants
  - Make SMS Height/Width available to remove magic numbers

Rust General
  - cargo clippy
  - profiling
        cargo install flamegraph

        cargo flamegraph
        #
        # Raspberry pi (ubuntu mate):
        # sudo apt-get install linux-tools-raspi
        #

  - remove all warnigns

vim setup
  Currently not really sure what the best way to setup vim is.  Generally, I like a 'minimal' setup, so I can easily get a consistent setup if plugins can't be used.

     It seems as though syntax highlighting works out of the box with Vim 8.1, and '!cargo fmt' seems to work well.

     my ~/.vimrc, (with rust additions):
        silent !stty -ixon

        " format for rust errors
        set efm=%.%#-->\ %f:%l:%c
        " format for git searches
        set efm+=%f:%l:%m
        
        au FileType rust set makeprg=cargo

        nnoremap <C-s> :cgetexpr system('git grep --recurse-submodules -n '. expand('<cword>'))<CR>

   Note to self, try these:
      git clone --depth 1 https://github.com/preservim/nerdtree.git  ~/.vim/pack/vendor/start/nerdtree
      git clone --depth 1 https://github.com/dense-analysis/ale.git ~/.vim/pack/git-plugins/start/ale
      git clone --depth 1 https://github.com/timonv/vim-cargo ~/.vim/pack/git-plugins/start/vim-cargo

Compilation errors:

SDL2:

      = note: /usr/bin/ld: cannot find -lSDL2
              collect2: error: ld returned 1 exit status
              
    error: could not compile `rustsega` due to previous error

Fix: Install SDL2:

perf setup (for flamegraph, see https://docs.kernel.org/admin-guide/perf-security.html):
   which perf
   # as root/sudo:
   cd /usr/bin
   ls -l perf 
   chgrp perf_users perf
   chmod o-rwx perf
   setcap "cap_perfmon,cap_sys_ptrace,cap_syslog=ep" perf 
   setcap -v "cap_perfmon,cap_sys_ptrace,cap_syslog=ep" perf 
   getcap perf 
   usermod -a -G perf_users <username>


