import { FunctionalComponent, h } from 'preact';
import * as style from './style.css';
import { useState } from 'preact/hooks';
import { Stop } from '../../types/Stop';
import { ApiClient } from '../../api/apiClient';
import { Modal } from '../../components/modal';

interface SubscribeModalProps {
  stop: Stop;
  onClose: () => void;
}

export const SubscribeModal: FunctionalComponent<SubscribeModalProps> = props => {
  const [email, setEmail] = useState<string>('');
  const apiClient = new ApiClient(API_URL);
  
  const handleSubscribe = (): void => {
    if (!/^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$/.test(email)) {
      console.log('Invalid!!');
    }
    apiClient.subscribe(email, props.stop.location_id);
  }

  if (!props.stop) {
    return <div></div>;
  }
  return (
    <Modal isOpen={true} onClickBackdrop={props.onClose}>
      <div className={style.modal}>
        <p>
          You&apos;re about to subscribe to e-mail notifications for{' '}
          <strong>{props.stop.street}</strong>.
        </p>
        <p>
          By subscribing you consent to that we will store your e-mail address
          and yada yada
        </p>
        <form className={style.form}>
          <input
            id="email"
            type="email"
            name="email"
            placeholder="E.g. example@email.com"
            value={email}
            required
            onInput={(event): void => {
              setEmail((event.target as HTMLInputElement).value);
            }}
          />
          <button
            onClick={(event): void => {
              event.preventDefault();
              handleSubscribe();
            }}
          >
            Subscribe!
          </button>
        </form>
      </div>
    </Modal>
  );
};
