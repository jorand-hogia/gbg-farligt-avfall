import axios, { AxiosInstance, AxiosResponse } from 'axios';
import { Stop } from '../types/Stop';

declare module 'axios' {
  // eslint-disable-next-line
  interface AxiosResponse<T = any> extends Promise<T> { }
}

export class ApiClient {
  private instance: AxiosInstance;

  public constructor(baseURL: string) {
    this.instance = axios.create({
      baseURL
    });
    this.initResponseInterceptor();
  }

  public getStops = (): Promise<Stop[]> => {
    return this.instance.get<Stop[]>('/stops').then(res => res.data);
  };

  public subscribe = (email: string, locationId: string): Promise<void> => {
    return this.instance.put('/subscriptions', {
      email,
      location_id: locationId
    }).then(res => res.data);
  }

  public verifySubscription = (email:string, authToken: string): Promise<void> => {
    return this.instance.post(`/subscriptions/verify?email=${email}&auth_token=${authToken}`)
      .then(res => res.data);
  }

  public removeSubscription = (email: string, unsubscribe_token: string): Promise<void> => {
    return this.instance.delete(`/subscriptions?email=${email}&unsubscribe_token=${unsubscribe_token}`)
      .then(res => res.data);
  }

  private initResponseInterceptor = (): void => {
    this.instance.interceptors.response.use(
      response => response,
      error => Promise.reject(error)
    );
  };
}
