config file
setup program

url parser:
GET requests: 
/range={dateStart}-{dateEnd}%20datapoint=

datapoint OPTIONAL (can be multiple values)

return array containing specified datapoints


POST requests:
Content-Type: application/json


future ideas:
REQUIRE AUTHORIZATION
introduce body size limits
make multiple file paths (would increase number of syscalls)
allow for mutliple output/input types like xml
add logs for changes that have been made


things to fix:
Reflecting data changes in the file and in the indexmap are asyncrounous, could lead to data inconsistincies
File synchronization is inefficient, appending to the end would be much easier