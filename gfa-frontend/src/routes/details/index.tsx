import { FunctionalComponent, h } from 'preact';
import { StopsContext } from '../../components/app';
import { useContext, useState } from 'preact/hooks';
import { ApiClient } from '../../api/apiClient';
import * as style from './style.css';

interface DetailsProps {
  locationId: string;
}

const capitalizeFirstLetter = (string: string): string => {
  return string.charAt(0).toUpperCase() + string.slice(1);
};

const handleSubscribe = (
  locationId: string,
  email: string
): Promise<string> => {
  if (
    !/^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$/.test(
      email
    )
  ) {
    return Promise.reject('Invalid e-mail address');
  }
  return new ApiClient(API_URL)
    .subscribe(email, locationId)
    .then(() => {
      return Promise.resolve('OK');
    })
    .catch(e => {
      return Promise.reject('Failed to subscribe');
    });
};

const Details: FunctionalComponent<DetailsProps> = props => {
  const { locationId } = props;
  const stops = useContext(StopsContext);
  const stop = stops.find(stop => stop.location_id === locationId);

  const [email, setEmail] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);

  return (
    <div className={style.main}>
      <div className={style.top}>{stop?.street}</div>
      <div className={style.details}>
        {stop?.description && <p>{capitalizeFirstLetter(stop.description)}.</p>}
      </div>
      <div className={style.instructions}>
        <p>
          To subscribe to e-mail notifications sent when the Göteborg Farligt
          Avfall truck will arrive to this street, enter your e-mail address
          below and hit subscribe.
        </p>
      </div>
      <form>
        <input
          id="email"
          type="email"
          placeholder="someone@somewhere.com"
          onInput={(event): void => {
            setEmail((event.target as HTMLInputElement).value);
          }}
        />
        <button
          onClick={(event): void => {
            event.preventDefault();
            setLoading(true);
            handleSubscribe(locationId, email)
              .then(() => {
                setLoading(false);
              })
              .catch(error => {
                setLoading(false);
                setError(error);
              });
          }}
        >
          Subscribe!
        </button>
      </form>
      {error && <div className={style.error}>{error}</div>}
    </div>
  );
};

export default Details;
