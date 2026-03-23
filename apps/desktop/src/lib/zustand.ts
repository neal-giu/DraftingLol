import { useSyncExternalStore } from 'react';

type Listener = () => void;
type Setter<T> = (partial: Partial<T> | ((state: T) => Partial<T>)) => void;
type Getter<T> = () => T;

export function create<T>(initializer: (set: Setter<T>, get: Getter<T>) => T) {
  let state: T;
  const listeners = new Set<Listener>();

  const getState: Getter<T> = () => state;
  const setState: Setter<T> = (partial) => {
    const nextPartial = typeof partial === 'function' ? partial(state) : partial;
    state = { ...state, ...nextPartial };
    listeners.forEach((listener) => listener());
  };

  state = initializer(setState, getState);

  function useStore(): T {
    return useSyncExternalStore(
      (listener) => {
        listeners.add(listener);
        return () => listeners.delete(listener);
      },
      getState,
      getState,
    );
  }

  useStore.getState = getState;
  useStore.setState = setState;

  return useStore;
}
