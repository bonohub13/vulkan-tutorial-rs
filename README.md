# Vulkan Tutorial in Rust + Ash

A rewrite of the [Vulkan Tutorial series by Brenden Galea](https://www.youtube.com/playlist?list=PL8327DO66nu9qYVKLDmdLW_84-yE4auCR) in Rust using the Ash crate.

## Why?
Out of pure curiosity, I wanted to learn Vulkan. \
But Instead of doing it the straight forward way, I felt like doing it in Rust.

## How does this work?
Each tag represents the codes in each videos. \
Ex) Tag `tutorial_01` matches the code in [Tutorial 01](https://www.youtube.com/watch?v=lr93-_cC8v4&list=PL8327DO66nu9qYVKLDmdLW_84-yE4auCR&index=2&t=10s&pp=iAQB).
#### How do you view each code corresponding to the videos?
For people who are unfamilliar with Git, this is how you switch between tags.
```
git checkout <tag name>
```
ex. If you want to switch over to `Tutorial_01`
```
git checktout tutorial_01
```

## How do you build this?
There are 2 ways you can build the codes.
1. Native
    1. Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
    2. Install Vulkan SDK
    3. Run the following command to build this
        ```
        make release
        ```
        - For building binary that are debuggable, run the following command
            ```
            make build
            ```
2. Docker container
    1. Install [docker](https://docs.docker.com/engine/install/) ([docker rootless](https://docs.docker.com/engine/security/rootless/) is preferred.)
    2. Install Vulkan SDK
    3. Run the following command to build the docker image
        ```
        make build-linux-image
        ```
    4. Run the following command to build this
        ```
        make docker-release
        ```
        - For building binary that are debuggable, run the following command
            ```
            make docker-build
            ```

## Wishes
Please feel free to file an issue if there are wierd implementation, bad codes, bugs, and etc. 
It really helps to keep this fresh and working. \
However, please **DO NOT** post requests of porting this to other Vulkan crates such as Vulkano, WGPU, etc. 
I want to keep this focused only on Ash since the structure of the library closely resembles that of Vulkan. \
That way, if I and others want to add changes or search for some implementation, the Vulkan documentation 
will be a viable resource. \
Other feature requests are fine and are welcome ;-)

## Finally
Greatest gratitude towards [Brenden Galea](https://www.youtube.com/@BrendanGalea) for such an awesome tutorial. \
Although being a complete NOOB for graphics API, it was very easy to watch and follow. 
If you happen to end up here and found this interesting, PLEASE consider subscribing to his channel. 
Without his tutorial, this would have never been possible.
