import { FunctionComponent, useEffect } from 'react';
import { useNavigate } from 'react-router';

import { Routes } from '@/types';

export const HomePage: FunctionComponent = () => {
  const navigate = useNavigate();

  useEffect(() => {
    navigate(Routes.Instances);
  }, [navigate]);

  return <></>;
};
