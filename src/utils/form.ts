export interface FormProps<T, R> {
  id?: number;
  initialValues: T;
  onSuccess?: (result: R) => void;
}
