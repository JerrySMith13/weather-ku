# KU Design Competition

# Portions:

weather-ku-console:
    a console application written in rust for the basic applications in the rules
    includes the mystery feature along with all other required features

weather-ku-api:
    the large API feature, written in rust and designed to work with only a CLI argument specifying what file contains the weather data
    can be run with cargo by using this command:
        ```cargo run -- (insertfile).txt```
weather-ku-web:
    a simple web application written with react and vite that is designed to work with the API. Also implements the small feature that draws data from a weather API.

sample-client:
    a small sample client to work with/test the API, written in node
