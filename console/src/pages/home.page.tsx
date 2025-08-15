import { FunctionComponent } from 'react';
import { useNavigate } from 'react-router';

import { Routes } from '@/types';

export const HomePage: FunctionComponent = () => {
  const navigate = useNavigate();

  navigate(Routes.Instances);

  return <></>;
};
