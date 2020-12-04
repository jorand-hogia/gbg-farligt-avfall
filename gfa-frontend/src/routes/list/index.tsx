import { FunctionalComponent, h } from 'preact';
import { Stop } from '../../types/Stop';
import { StopListItem } from '../../components/stop-list-item';
import { StopsContext } from '../../components/app';
import { SubscribeModal } from '../../components/subscribe-modal';
import * as style from './style.css';
import { useContext, useState } from 'preact/hooks';

const List: FunctionalComponent<{}> = () => {
  const stops = useContext(StopsContext);

  const [showSubscribe, setShowSubscribe] = useState<boolean>(false);
  const [stopToSubscribeTo, setStopToSubscribeTo] = useState<Stop | null>(null);

  const onSubscribe = (stop: Stop): void => {
    setStopToSubscribeTo(stop);
    setShowSubscribe(true);
  };

  const onCloseSubscribeModal = (): void => {
    setShowSubscribe(false);
    setStopToSubscribeTo(null);
  };

  return (
    <div className={style.main}>
      <div>
        {stops.map(stop => (
          <StopListItem
            stop={stop}
            key={stop.location_id}
            onSubscribe={onSubscribe}
          />
        ))}
      </div>
      <div>
        <p>
          Coordinates for each location are powered by {}
          <a href="http://www.mapquest.com">MapQuest</a>
        </p>
      </div>
      {showSubscribe && stopToSubscribeTo && (
        <SubscribeModal
          stop={stopToSubscribeTo}
          onClose={onCloseSubscribeModal}
        />
      )}
    </div>
  );
};

export default List;
