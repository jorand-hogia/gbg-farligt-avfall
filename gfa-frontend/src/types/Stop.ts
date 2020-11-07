import { Coordinate } from './Coordinate';

export type Stop = {
  location_id: string;
  street: string;
  district: string;
  description: string;
  coordinate?: Coordinate;
};
