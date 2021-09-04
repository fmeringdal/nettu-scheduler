import { BusyCalendar, Service, TimePlan } from "./domain/service";
import { NettuBaseClient } from "./baseClient";
import { Metadata } from "./domain/metadata";

type AddUserToServiceRequest = {
  userId: string;
  availability?: TimePlan;
  bufferBefore?: number;
  bufferAfter?: number;
  closestBookingTime?: number;
  furthestBookingTime?: number;
};

type UpdateUserToServiceRequest = {
  userId: string;
  availability?: TimePlan;
  bufferBefore?: number;
  bufferAfter?: number;
  closestBookingTime?: number;
  furthestBookingTime?: number;
};

type GetServiceBookingslotsReq = {
  ianaTz: string;
  duration: number;
  interval: number;
  startDate: string;
  endDate: string;
  userIds?: string[];
};

type ServiceBookingSlot = {
  start: number;
  duration: number;
  userIds: string[];
};

type GetServiceBookingslotsResponse = {
  dates: {
    date: string;
    slots: ServiceBookingSlot[];
  }[];
};

type CreateServiceRequest = {
  metadata?: Metadata;
};

type UpdateServiceRequest = {
  metadata?: Metadata;
};

type ServiceResponse = {
  service: Service;
};

type AddBusyCalendar = {
  serviceId: string;
  userId: string;
  calendar: BusyCalendar;
};

type RemoveBusyCalendar = {
  serviceId: string;
  userId: string;
  calendar: BusyCalendar;
};

export class NettuServiceClient extends NettuBaseClient {
  public create(data?: CreateServiceRequest) {
    data = data ? data : {};
    return this.post<ServiceResponse>("/service", data);
  }

  public update(serviceId: string, data?: UpdateServiceRequest) {
    data = data ? data : {};
    return this.put<ServiceResponse>(`/service/${serviceId}`, data);
  }

  public find(serviceId: string) {
    return this.get<Service>(`/service/${serviceId}`);
  }

  public remove(serviceId: string) {
    return this.delete<ServiceResponse>(`/service/${serviceId}`);
  }

  public addUser(serviceId: string, data: AddUserToServiceRequest) {
    return this.post<ServiceResponse>(`/service/${serviceId}/users`, data);
  }

  public removeUser(serviceId: string, userId: string) {
    return this.delete<ServiceResponse>(
      `/service/${serviceId}/users/${userId}`
    );
  }

  public updateUserInService(
    serviceId: string,
    data: UpdateUserToServiceRequest
  ) {
    return this.put<ServiceResponse>(
      `/service/${serviceId}/users/${data.userId}`,
      data
    );
  }

  public getBookingslots(serviceId: string, req: GetServiceBookingslotsReq) {
    return this.get<GetServiceBookingslotsResponse>(
      `/service/${serviceId}/booking?${getBookingslotsQueryString(req)}`
    );
  }

  public addBusyCalendar(input: AddBusyCalendar) {
    return this.put<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.calendar,
      }
    );
  }

  public removeBusyCalendar(input: RemoveBusyCalendar) {
    return this.deleteWithBody<string>(
      `/service/${input.serviceId}/users/${input.userId}/busy`,
      {
        busy: input.calendar,
      }
    );
  }
}

const getBookingslotsQueryString = (req: GetServiceBookingslotsReq): string => {
  let qs = `startDate=${req.startDate}&endDate=${req.endDate}&ianaTz=${req.ianaTz}&duration=${req.duration}&interval=${req.interval}`;
  if (req.userIds) {
    qs += `&hostUserIds=${req.userIds.join(",")}`;
  }

  return qs;
};

export class NettuServiceUserClient extends NettuBaseClient {
  public getBookingslots(serviceId: string, req: GetServiceBookingslotsReq) {
    return this.get<GetServiceBookingslotsResponse>(
      `/service/${serviceId}/booking?${getBookingslotsQueryString(req)}`
    );
  }
}
