import { h, FunctionalComponent } from 'preact';
import { route } from 'preact-router';
import { Stop } from '../../types/Stop';
import * as style from './style.css';

interface StopListItemProps {
  stop: Stop;
}

export const StopListItem: FunctionalComponent<StopListItemProps> = ({
  stop
}) => {
  return (
    <div
      className={style.stop}
      onClick={() => {
        route(`/stop/${stop.location_id}`);
      }}
    >
      <div>
        <div className={style.title}>{stop.street}</div>
        <div className={style.subtitle}>{stop.district}</div>
      </div>
    </div>
  );
};
