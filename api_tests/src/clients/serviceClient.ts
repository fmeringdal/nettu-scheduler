import { NettuBaseClient } from "./baseClient";

type CreatedServiceResponse = {
  serviceId: string;
};

type AddUserToServiceRequest = {
  userId: string;
  calendarIds: string[];
};

type UpdateUserToServiceRequest = {
  userId: string;
  calendarIds: string[];
};

export type GetServiceBookingslotsReq = {
  ianaTz: string;
  duration: number;
  interval: number;
  date: string;
};

export type ServiceBookingSlot = {
  start: number;
  duration: number;
  userIds: string[];
};

export type GetServiceBookingslotsResponse = {
  bookingSlots: ServiceBookingSlot[];
};

export class NettuServiceClient extends NettuBaseClient {
  public insert() {
    return this.post<CreatedServiceResponse>("/service", undefined);
  }

  public addUserToService(serviceId: string, data: AddUserToServiceRequest) {
    return this.post<void>(`/service/${serviceId}/users`, data);
  }

  public removeUserFromService(serviceId: string, userId: string) {
    return this.delete<void>(`/service/${serviceId}/users/${userId}`);
  }

  public updateUserInService(
    serviceId: string,
    data: UpdateUserToServiceRequest
  ) {
    return this.put<void>(`/service/${serviceId}/users/${data.userId}`, {
      calendarIds: data.calendarIds,
    });
  }

  public getBookingslots(serviceId: string, req: GetServiceBookingslotsReq) {
    const queryString = `date=${req.date}&ianaTz=${req.ianaTz}&duration=${req.duration}&interval=${req.interval}`;
    return this.get<GetServiceBookingslotsResponse>(
      `/service/${serviceId}/booking?${queryString}`
    );
  }
}
