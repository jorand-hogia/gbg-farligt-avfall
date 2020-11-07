import { FunctionalComponent, h } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import { Stop } from '../../types/Stop';
import { StopComponent } from '../../components/stop';
import { ApiClient } from '../../api/apiClient';
import * as style from './style.css';

const Home: FunctionalComponent = () => {
  const [stops, setStops] = useState<Stop[]>([]);
  useEffect(() => {
    const apiClient = new ApiClient(API_URL[0]);
    apiClient
      .getStops()
      .then(stops => {
        setStops(stops);
      })
      .catch(err => {
        console.log('I am error');
        console.error(err);
      });
  }, []);

  return (
    <div className={style.home}>
      <div>
        {stops.map(stop => (
          <StopComponent stop={stop} key={stop.location_id} />
        ))}
      </div>
      <div>
        <p>
          Coordinates for each location are powered by {}
          <a href="http://www.mapquest.com">MapQuest</a>
        </p>
      </div>
    </div>
  );
};

export default Home;
