# Project Goals
The goal of this project is to design a firmware for the ESP32-C6 that can be used to communicate with WiFi as well as Thread networks from a secondary controller over SPI. This firmware should handle as much of the required code as possible to interact with these networks and expose an extremely simple API to a secondary Microcontroller. The firmware should be designed to be as modular as possible, allowing for easy updates and changes in the future. The firmware should also be designed to be as efficient as possible, minimizing the amount of code that needs to be run on the ESP32-C6 itself. The firmware should also be designed to be as flexible as possible, allowing for easy integration with other systems and components.

## Project Specifics
- This project uses `esp-alloc` for allocations, do not use `std` or `alloc` directly.

## Required Networking Tasks
- Connect the ESP32-C6 to a WiFi network
- Connect the ESP32-C6 to a Thread network
- Connect to a TCP server over WiFi
- Connect to a TCP server over Thread
- Connect to a UDP server over WiFi
- Connect to a UDP server over Thread
- Send and receive data over the TCP connection
- Send and receive data over the UDP connection
- Send and receive data over the Thread connection
- Send and receive data over the WiFi connection
- Expose HTTP server for configuration
- Expose API endpoints for configuration and status


## Required SPI API
- Initialize the SPI connection
- Maintain a coherent SPI API that is simple to use
- Send and receive data over the SPI connection
