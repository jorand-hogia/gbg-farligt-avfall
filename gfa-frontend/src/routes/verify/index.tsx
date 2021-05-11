import { FunctionalComponent, h } from 'preact';
import { useState } from 'preact/hooks';
import * as style from './style.css';
import { ApiClient } from '../../api/apiClient';

interface VerifySubscriptionProps {
    auth_token: string,
    email: string,
}

const VerifySubscription: FunctionalComponent<VerifySubscriptionProps> = (props: VerifySubscriptionProps) => {
    const [response, setResponse] = useState<string>('');

    const { auth_token: authToken, email } = props;

    const apiClient = new ApiClient(API_URL);

    const verifySubscription = (): void => {
        apiClient.verifySubscription(email, authToken)
            .then(res => {
                setResponse('Successfully verified subscription')
                console.log(res);
            })
            .catch(err => {
                console.error(err);
                setResponse('Failed to verify subscription');
            });
    }

    return (
        <div className={style.main}>
            {response && <p>{response}</p>}
            {!response && !(authToken && email) && <p>Missing authentication token or email</p>}
            {!response && authToken && email && <p>Click <span onClick={() => verifySubscription()} className={style.link}>here</span> to confirm your subscription</p>}
        </div>
    );

}

export default VerifySubscription;