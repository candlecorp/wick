export const BATCH_SIGNATURE = {
  inputs: {
    inputs: {
      type: 'list',
      element: {
        type: 'internal',
        id: '__input__',
      },
    },
  },
  outputs: {
    result: {
      type: 'bool',
    },
  },
};

export const BATCH_COMPONENT_NAME = '__batch__';
