import native from './native'

export const getApplicantId = (interval: string | number, start: string | number, end: string | number, model_id: string) => native.get_applicant_id(interval, start, end, model_id);
