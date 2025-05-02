import { useDispatch, useSelector } from "react-redux";
import type { AppDispatch, AppState } from "@/store";

export const useAppDispatch = useDispatch.withTypes<AppDispatch>();
export const useAppSelector = useSelector.withTypes<AppState>();
