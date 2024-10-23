import { useState } from 'react';
import './App.css'
import { WeatherData, Options, getDataFromServer } from './util/WeatherData';

function getDataFromAPI(xCoord: number, yCoord: number){

}

function DateDisplay({ data }: { data: WeatherData[] }){
  return(
    <div id="tableDisplay">
      <table id="weatherDataTable">
        <thead>
          <tr>
          <th>Date</th>
          <th>Weather Code</th>
          <th>Max Temperature</th>
          <th>Min Temperature</th>
          <th>Precipitation Sum</th>
          <th>Max Wind Speed</th>
          <th>Precipitation Probability Max</th>
          </tr>
        </thead>
        <tbody>
          
            {data.map((entry, index) => (
            <tr key={index}>
              <th scope='row'>{entry.date}</th>
              <td>{entry.weather_code}</td>
              <td>{entry.temperature_max}</td>
              <td>{entry.temperature_min}</td>
              <td>{entry.precipitation_sum}</td>
              <td>{entry.wind_speed_max}</td>
              <td>{entry.precipitation_probability_max}</td>

            </tr>
            ))}
        </tbody>
      </table>
    </div>
  )
}

function App() {
  const [date1, setDate1] = useState('');
  const [date2, setDate2] = useState('');
  const [options, setOptions] = useState([]);
  const [data, setData] = useState([] as WeatherData[]);
  const [display, setDisplay] = useState(false);
  const [graphOption, setGraph] = useState(Options.TempMax); 
  return (
    <>
    <div id='dateSelect'>
      <p>Start date: </p>
      <input type="date" onChange={e => {setDate1(e.target.value); console.log(e.target.value)}}></input>
      <p>End date: </p>
      <input type="date" onChange={e => {setDate2(e.target.value); console.log(e.target.value)}}></input>
      <p>{date1} to {date2}</p>
      <button onClick={async e => {
        let data = await getDataFromServer([date1, date2]);
        setData(data);
        setDisplay(true);
      }}>CLICK ME</button>
    </div>
    {display && (
    <div id="data">
    <DateDisplay data={data} />
    </div>
    )}
    
    </>
  )
}

export default App
