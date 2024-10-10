import { useState } from 'react';
import './App.css'

enum Options {
  WeatherCode = "weather_code",
  TempMax = "temperature_max",
  TempMin = "temperature_min",
  PrecipSum = "precipitation_sum",
  WindMax = "wind_speed_max",
  PrecipProbMax = "precipitation_probability_max",
}

const addr = 'http://localhost:3000';

function getDataFromLocal(dates: String[], options: Options[] = []) {
  let url = addr + '/q?date=';
  for (let i = 0; i < dates.length; i++) {
    url += dates[i];
    if (i < dates.length - 1) {
      url += '%20';
    }
  }
  if (options.length > 0) {
    url += '&values=';
    for (let i = 0; i < options.length; i++) {
      url += options[i];
      if (i < options.length - 1) {
        url += '%20';
      }
    }
  }
  fetch(url)
    .then(res => res.json())
    .then(data => {console.log(data); return data});
}

function App() {
  const [dates, setDates] = useState([]);
  const [options, setOptions] = useState([]);
  return (
    <>

    </>
  )
}

export default App
