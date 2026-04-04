import {
  createContext,
  useContext,
  useReducer,
  type Dispatch,
  type ReactNode,
  createElement,
} from "react";
import type { WasmDocument } from "./wasm-bridge";

export interface DocumentEntry {
  id: string;
  name: string;
  bytes: Uint8Array;
  doc: WasmDocument | null;
  format: string;
  size: number;
}

export interface StoreState {
  documents: Map<string, DocumentEntry>;
  activeDocumentId: string | null;
}

export type StoreAction =
  | { type: "ADD_DOCUMENT"; payload: DocumentEntry }
  | { type: "REMOVE_DOCUMENT"; payload: string }
  | { type: "SET_ACTIVE"; payload: string | null };

function storeReducer(state: StoreState, action: StoreAction): StoreState {
  switch (action.type) {
    case "ADD_DOCUMENT": {
      const next = new Map(state.documents);
      next.set(action.payload.id, action.payload);
      return {
        ...state,
        documents: next,
        activeDocumentId: action.payload.id,
      };
    }
    case "REMOVE_DOCUMENT": {
      const next = new Map(state.documents);
      const entry = next.get(action.payload);
      if (entry?.doc) {
        try {
          entry.doc.free();
        } catch {
          // already freed
        }
      }
      next.delete(action.payload);
      return {
        ...state,
        documents: next,
        activeDocumentId:
          state.activeDocumentId === action.payload
            ? (next.keys().next().value ?? null)
            : state.activeDocumentId,
      };
    }
    case "SET_ACTIVE":
      return { ...state, activeDocumentId: action.payload };
    default:
      return state;
  }
}

const initialState: StoreState = {
  documents: new Map(),
  activeDocumentId: null,
};

const StoreContext = createContext<StoreState>(initialState);
const DispatchContext = createContext<Dispatch<StoreAction>>(() => {});

export function StoreProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(storeReducer, initialState);

  return createElement(
    StoreContext.Provider,
    { value: state },
    createElement(DispatchContext.Provider, { value: dispatch }, children),
  );
}

export function useStore(): [StoreState, Dispatch<StoreAction>] {
  return [useContext(StoreContext), useContext(DispatchContext)];
}

export function useActiveDocument(): DocumentEntry | null {
  const [state] = useStore();
  if (!state.activeDocumentId) return null;
  return state.documents.get(state.activeDocumentId) ?? null;
}
