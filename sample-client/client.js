const fs = require('fs');

// Define the URL to fetch
const url = 'http://localhost:3000/q?dates=2024-4-27';

// Define the JSON array to send in the POST request


// Fetch the URL with a POST request
fetch(url, {
    method: 'DELETE'
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