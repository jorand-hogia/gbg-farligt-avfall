import { h, FunctionalComponent } from 'preact';
import { Stop } from '../../types/Stop';

interface StopComponentProps {
  stop: Stop;
}

export const StopComponent: FunctionalComponent<StopComponentProps> = ({
  stop
}) => {
  return (
    <p>
      {stop.district}, {stop.street}
    </p>
  );
};
