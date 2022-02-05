
<!-- README.md is generated from README.Rmd. Please edit that file -->

# wgpugd: An WebGPU Graphics Device for R

<!-- badges: start -->
<!-- badges: end -->

## Overview

### What is WebGPU?

[WebGPU](https://www.w3.org/TR/webgpu/) is an API that exposes the
capabilities of GPU hardware.

### What is wgpu?

There are two major libraries that implement WebGPU standard; one is
[Dawn](https://dawn.googlesource.com/dawn) (used in Chrome), and the
other one is [wgpu](https://wgpu.rs/), a pure-Rust implementation.

wgpu is what’s behind the WebGPU support of Firefox and Deno, and is
used in many places in the Rust’s graphics ecosystem.

### Why WebGPU for R?

The main motivation is to add post-effect to graphics with [WebGPU
Shader Language (WGSL)](https://www.w3.org/TR/WGSL/%3E). But of course
the power of GPU should simply contribute to the high-performance.

## Installation

You can install the development version of wgpugd like so:

``` r
devtools::install_github("yutannihilation/wgpugd")
```
