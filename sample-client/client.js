const fs = require('fs');

// Define the URL to fetch
const url = 'http://localhost:3000';

// Define the JSON array to send in the POST request
const data = [
    { date: "2024-04-22", weather_code: 100, temperature_max: 25.0, temperature_min: 15.0, precipitation_sum: 5.0, wind_speed_max: 10.0, precipitation_probability_max: 80.0 },
    { date: "2024-04-23", weather_code: 200, temperature_max: 25.0, temperature_min: 15.0, precipitation_sum: 5.0, wind_speed_max: 10.0, precipitation_probability_max: 80.0 },

];

// Fetch the URL with a POST request
fetch(url, {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify(data)
})
    .then(response => {
        if (!response.ok) {
            return response.text().then(errorBody => {
                throw new Error(`HTTP error: ${response.status}, Body: ${errorBody}`);
            });
        }
        return response.text(); // Use response.json() if the response is JSON
    })
    .then(body => {
        console.log(body); // Print the response body

        // Write the response body to a file
        fs.writeFile('response.json', body, (err) => {
            if (err) {
                console.error('Error writing to file', err);
            } else {
                console.log('Response written to response.json');
            }
        });
    })
    .catch(error => {
        console.error('Fetch error:', error.message);
    });