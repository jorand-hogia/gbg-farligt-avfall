import { FunctionalComponent, h } from 'preact';
import { StopListItem } from '../../components/stop-list-item';
import { StopsContext } from '../../components/app';
import * as style from './style.css';
import { useContext } from 'preact/hooks';

const List: FunctionalComponent<{}> = () => {
  const stops = useContext(StopsContext);

  return (
    <div className={style.main}>
      <div className={style.intro}>
        <h1>Greetings!</h1>
        <p>
          Here you can subscribe to e-mail notifications when the Göteborg
          Farligt Avfall truck is about to arrive. Note that the notifications
          are provided as a best effort and that we are in no way affiliated
          with Göteborg stad.
        </p>
        <p>Click on one of the locations below to subscribe.</p>
      </div>
      <div className={style.list}>
        {stops.map(stop => (
          <StopListItem stop={stop} key={stop.location_id} />
        ))}
      </div>
    </div>
  );
};

export default List;
