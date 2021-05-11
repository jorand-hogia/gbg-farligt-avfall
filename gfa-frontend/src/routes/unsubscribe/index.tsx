import { FunctionalComponent, h } from 'preact';
import { useState } from 'preact/hooks';
import { ApiClient } from '../../api/apiClient';
import * as style from './style.css';

interface UnsubscribeProps {
    email: string,
    unsubscribe_token: string,
}

const Unsubscribe: FunctionalComponent<UnsubscribeProps> = (props: UnsubscribeProps) => {
    const [response, setResponse] = useState<string>('');
    const {email, unsubscribe_token} = props;
    const apiClient = new ApiClient(API_URL);

    const removeSubscription = (): void => {
        apiClient.removeSubscription(email, unsubscribe_token)
            .then(() => {
                setResponse('Successfully unsubscribed')
            })
            .catch(err => {
                console.error(err);
                setResponse('Failed to unsubscribe');
            });
    }

    return (
        <div className={style.main}>
            {response && <p>{response}</p>}
            {!response && !(email && unsubscribe_token) && <p>Missing email or unsubscribe token</p>}
            {!response && email && unsubscribe_token && <p>Click <span onClick={() => removeSubscription()} className={style.link}>here</span> to unsubscribe</p>}
        </div>
    );
}

export default Unsubscribe;
