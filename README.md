
<!-- README.md is generated from README.Rmd. Please edit that file -->

# wgpugd: A WebGPU Graphics Device for R

<!-- badges: start -->
<!-- badges: end -->

## Overview

### What is WebGPU?

[WebGPU](https://www.w3.org/TR/webgpu/) is an API that exposes the
capabilities of GPU hardware.

### What is wgpu?

There are two major libraries that implement the WebGPU standard; one is
[Dawn](https://dawn.googlesource.com/dawn) (used in Chrome), and the
other one is [wgpu](https://wgpu.rs/), a pure-Rust implementation.

wgpu is what’s behind the WebGPU support of Firefox and Deno, and is
used in many places in the Rust’s graphics ecosystem.

### Why WebGPU for R?

The main motivation is to add post-effect to graphics with [WebGPU
Shader Language (WGSL)](https://www.w3.org/TR/WGSL/%3E). But, of course,
the power of GPU should simply contribute to high performance!

## Installation

You can install the development version of wgpugd like so:

``` r
devtools::install_github("yutannihilation/wgpugd")
```

## Usages

:warning: wgpugd is currently at its verrry early stage of the
development! :warning:

``` r
library(wgpugd)

wgpugd(10, 10)
hist(1:100)
dev.off()
#> [DEBUG] vertex: [Vertex { position: [0.8511112, 0.696], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.8511112, 0.896], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.77111113, 0.896], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.77111113, 0.696], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.6711111, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.87111115, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.87111115, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.6711111, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.34666666, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.5466667, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.5466667, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.34666666, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.022222184, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.22222218, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.22222218, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.022222184, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.30222222, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.10222223, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.10222223, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.30222222, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.62666667, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.42666665, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.42666665, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.62666667, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.9511112, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.75111115, 0.8160001], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.75111115, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [0.9511112, 0.796], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.936, -0.77555555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.73599994, -0.77555555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.73599994, 0.7355555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.936, 0.7355555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, 0.83555555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, 0.6355555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, 0.6355555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, 0.83555555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, 0.5333334], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, 0.3333334], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, 0.3333334], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, 0.5333334], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, 0.23111114], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, 0.031111144], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, 0.031111144], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, 0.23111114], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, -0.071111105], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, -0.2711111], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, -0.2711111], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, -0.071111105], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, -0.3733333], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, -0.5733333], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, -0.5733333], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, -0.3733333], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, -0.6755555], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.856, -0.8755556], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, -0.8755556], color: [0.0, 0.0, 0.0, 1.0] }, Vertex { position: [-0.83599997, -0.6755555], color: [0.0, 0.0, 0.0, 1.0] }]
#> [DEBUG] index: [0, 1, 2, 1, 3, 2, 2, 3, 0, 3, 1, 0, 4, 5, 6, 5, 7, 6, 6, 7, 4, 7, 5, 4, 8, 9, 10, 9, 11, 10, 10, 11, 8, 11, 9, 8, 12, 13, 14, 13, 15, 14, 14, 15, 12, 15, 13, 12, 16, 17, 18, 17, 19, 18, 18, 19, 16, 19, 17, 16, 20, 21, 22, 21, 23, 22, 22, 23, 20, 23, 21, 20, 24, 25, 26, 25, 27, 26, 26, 27, 24, 27, 25, 24, 28, 29, 30, 29, 31, 30, 30, 31, 28, 31, 29, 28, 32, 33, 34, 33, 35, 34, 34, 35, 32, 35, 33, 32, 36, 37, 38, 37, 39, 38, 38, 39, 36, 39, 37, 36, 40, 41, 42, 41, 43, 42, 42, 43, 40, 43, 41, 40, 44, 45, 46, 45, 47, 46, 46, 47, 44, 47, 45, 44, 48, 49, 50, 49, 51, 50, 50, 51, 48, 51, 49, 48, 52, 53, 54, 53, 55, 54, 54, 55, 52, 55, 53, 52]
#> png 
#>   2

knitr::include_graphics("tmp_wgpugd.png")
```

<img src="tmp_wgpugd.png" width="100%" />

## References

-   wgpugd uses [extendr](https://extendr.github.io/), a Rust extension
    mechanism for R, both to communicate with the actual graphics device
    implementation in Rust from R, and to access R’s graphics API from
    Rust.
-   If you are curious about developing a Rust program with wgpu, I’d
    recommend [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) to get
    started.
-   [lyon](https://github.com/nical/lyon) is a library for “path
    tessellation,” which is necessary to draw lines on GPU.
