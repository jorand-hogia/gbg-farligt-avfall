import { FunctionalComponent, h } from 'preact';
import { useContext } from 'preact/hooks';
import { StopsContext } from '../../components/app';
import * as style from './style.css';

interface StopProps {
  id: string;
}

const Stop: FunctionalComponent<StopProps> = ({ id }) => {
  const stops = useContext(StopsContext);
  const stop = stops.filter(stop => stop.location_id === id)[0];

  if (!stop) {
    return <div></div>;
  }

  return (
    <div className={style.main}>
      <div className={style.title}>{stop.street}</div>
      <div className={style.subtitle}>{stop.district}</div>
      <div className={style.description}>{stop.description}</div>
    </div>
  );
};

export default Stop;
