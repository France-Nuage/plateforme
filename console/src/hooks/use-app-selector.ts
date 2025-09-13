import { useSelector } from 'react-redux';

import type { AppState } from '@/store';

export const useAppSelector = useSelector.withTypes<AppState>();
