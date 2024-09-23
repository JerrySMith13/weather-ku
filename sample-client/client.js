const fs = require('fs');

// Define the URL to fetch
const url = 'http://localhost:3000';

// Define the JSON array to send in the POST request
const data = [
    { date: "2024-04-26", weather_code: 100, temp_max: 25.0, temp_min: 15.0, precip_sum: 5.0, max_wind: 10.0, precip_prob_max: 80.0 },
    { date: "2024-04-27", weather_code: 200, temp_max: 22.0, temp_min: 14.0, precip_sum: 3.0, max_wind: 8.0, precip_prob_max: 60.0 }
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
            throw new Error(`HTTP error! status: ${response.status}`);
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
        console.error('Fetch error:', error);
    });