export enum Options {
    WeatherCode = "weather_code",
    TempMax = "temperature_max",
    TempMin = "temperature_min",
    PrecipSum = "precipitation_sum",
    WindMax = "wind_speed_max",
    PrecipProbMax = "precipitation_probability_max",
}

export interface WeatherData {
    date: String;
    weather_code: number;
    temperature_max: number
    temperature_min: number
    precipitation_sum: number
    wind_speed_max: number
    precipitation_probability_max: number
}

const addr = 'http://localhost:3000';

export async function getDataFromServer(dates: [String, String], options: Options[] = []): Promise<WeatherData[]> {
    let url = addr + '/q?dates=';
    if (compDate(dates[0], dates[1]) == false) {
        return [];
    }
    else {
        url += dates[0] + '%20' + dates[1];
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
    console.log(url);
    const req = await fetch(url);
    if (!req.ok){
        console.log(req.status)
        console.log(req.statusText)
        console.log(req.body)
    }
    const json = req.json();
    return json
}




function compDate(first: String, second: String) {
    let first_as_num = 0;
    let first_year = parseInt(first.substring(0, 4));
    let first_month = parseInt(first.substring(5, 7));
    let days = parseInt(first.substring(8));
    first_as_num += ((first_year - 1) * 365) + ((first_month - 1) * 30) + days

    let sec_as_num = 0;
    let sec_year = parseInt(second.substring(0, 4));
    let sec_month = parseInt(second.substring(5, 7));
    let sec_days = parseInt(second.substring(8));
    sec_as_num += ((sec_year - 1) * 365) + ((sec_month - 1) * 30) + sec_days;
    if (first_as_num > sec_as_num) return false;
    else return true;

}
