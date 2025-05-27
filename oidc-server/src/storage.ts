import { User } from "./types";

/**
 * In-memory user stores.
 */
export const users = new Map<string, User>();

/**
 * Create a new user.
 */
export const createUser = (username: string, profile: any) => {
  users.set(username, { sub: username, ...profile });
};

/**
 * Retrieve a user by its sub.
 */
export const findUserBySub = (sub: string) => {
  return users.get(sub);
};
