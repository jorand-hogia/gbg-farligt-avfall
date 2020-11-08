import { FunctionalComponent, h } from 'preact';
import { StopListItem } from '../../components/stop-list-item';
import { StopsContext } from '../../components/app';
import { Stop } from '../../types/Stop';
import * as style from './style.css';
import { useContext } from 'preact/hooks';

const List: FunctionalComponent<{}> = () => {
  const stops = useContext(StopsContext);
  return (
    <div className={style.home}>
      <div>
        {stops.map(stop => (
          <StopListItem stop={stop} key={stop.location_id} />
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

export default List;
