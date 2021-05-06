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
      onClick={() => route(`/details/${stop.location_id}`)}
    >
      <div className={style.primary}>{stop.street}</div>
      <div className={style.meta}>{stop.district}</div>
    </div>
  );
};
