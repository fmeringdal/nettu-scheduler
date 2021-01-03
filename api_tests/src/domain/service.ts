export type UserServiceResource = {
  id: string;
  userId: string;
  calendarIds: string[];
};

export type Service = {
  id: string;
  accountId: string;
  users: UserServiceResource[];
};
