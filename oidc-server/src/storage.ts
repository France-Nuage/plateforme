import { User } from "./types";

/**
 * In-memory user stores.
 */
export const users = new Map<string, User>();

/**
 * Create a new user.
 */
export const createUser = (user: User) => {
  users.set(user.username, user);
};

/**
 * Retrieve a user by its sub.
 */
export const findUserBySub = (sub: string): User | undefined => {
  return users.get(sub);
};
