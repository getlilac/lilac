import { zodResolver } from '@hookform/resolvers/zod';
import * as React from 'react';
import { useForm, useFormContext } from 'react-hook-form';
import { z } from 'zod/v4';

import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Separator } from '@/components/ui/separator';
import { defineStepper } from '@stepperize/react';
import createFormStore from '@/store/use-form-data';
import { Input } from '@/components/ui/input';

const clusterSchema = z.object({
  clusterName: z
    .string({
      error: 'Cluster name is required.',
    })
    .min(1, { message: 'Cluster name cannot be empty.' }),
  clusterDescription: z.string().optional(),
});

const useFormStore = createFormStore<z.infer<typeof clusterSchema>>();

export type ClusterFormValues = z.infer<typeof clusterSchema>;

const { useStepper, steps, utils } = defineStepper({
  id: 'configureCluster',
  label: 'Cluster Details',
  schema: clusterSchema,
});

export interface CreateClusterFormProps {
  onSubmit: (values: ClusterFormValues) => void;
}

export function CreateClusterForm(props: CreateClusterFormProps) {
  const stepper = useStepper();
  const { setFormValues } = useFormStore((state) => ({
    formValues: state.formValues,
    setFormValues: state.setFormValues,
  }));
  const form = useForm({
    mode: 'onChange',
    resolver: zodResolver(stepper.current.schema),
  });

  const onSubmit = (values: z.infer<typeof stepper.current.schema>) => {
    if (stepper.isLast) {
      props.onSubmit(values as ClusterFormValues);
      stepper.reset();
    } else {
      setFormValues(values);
      stepper.next();
    }
  };

  const currentIndex = utils.getIndex(stepper.current.id);

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <nav aria-label='Create Clusters Steps' className='group my-4'>
          <ol className='flex flex-col gap-2' aria-orientation='vertical'>
            {stepper.all.map((step, index, array) => (
              <React.Fragment key={step.id}>
                <li className='flex flex-shrink-0 items-center gap-4'>
                  <Button
                    type='button'
                    role='tab'
                    variant={index <= currentIndex ? 'default' : 'secondary'}
                    aria-current={
                      stepper.current.id === step.id ? 'step' : undefined
                    }
                    aria-posinset={index + 1}
                    aria-setsize={steps.length}
                    aria-selected={stepper.current.id === step.id}
                    className='flex size-10 items-center justify-center rounded-full'
                    onClick={async () => {
                      if (index <= currentIndex) {
                        stepper.goTo(step.id);
                      }
                    }}
                  >
                    {index + 1}
                  </Button>
                  <span className='text-sm font-medium'>{step.label}</span>
                </li>
                <div className='flex gap-4'>
                  <div
                    className='flex justify-center'
                    style={{
                      paddingInlineStart: '1.25rem',
                    }}
                  >
                    {index < array.length - 1 && (
                      <Separator
                        orientation='vertical'
                        className={`h-full w-[1px] ${
                          index < currentIndex
                            ? 'bg-accent'
                            : 'bg-accent-border'
                        }`}
                      />
                    )}
                  </div>
                  <div className='my-4 flex-1'>
                    {stepper.current.id === step.id &&
                      stepper.switch({
                        configureCluster: () => <ConfigureClusterDetails />,
                      })}
                  </div>
                </div>
              </React.Fragment>
            ))}
          </ol>
        </nav>
        <div className='space-y-4'>
          <div className='flex justify-end gap-4'>
            <Button
              variant='secondary'
              onClick={stepper.prev}
              disabled={stepper.isFirst}
              type='button'
            >
              Back
            </Button>
            <Button type='submit'>{stepper.isLast ? 'Submit' : 'Next'}</Button>
          </div>
        </div>
      </form>
    </Form>
  );
}

function ConfigureClusterDetails() {
  const { register } = useFormContext<ClusterFormValues>();

  return (
    <div className='space-y-4 text-start'>
      <FormField
        name={register('clusterName').name}
        render={({ field }) => {
          return (
            <FormItem>
              <FormLabel>Name</FormLabel>
              <FormDescription>
                A friendly name for your cluster
              </FormDescription>
              <FormMessage />
              <FormControl>
                <Input {...field} placeholder='cluster-1'></Input>
              </FormControl>
            </FormItem>
          );
        }}
      />
      <FormField
        name={register('clusterDescription').name}
        render={({ field }) => {
          return (
            <FormItem>
              <FormLabel>
                Description
                <span className='text-gray-text-muted text-xs font-light italic'>
                  optional
                </span>
              </FormLabel>
              <FormDescription>A description of your cluster</FormDescription>
              <FormControl>
                <Input {...field} placeholder='Cluster description...'></Input>
              </FormControl>
              <FormMessage />
            </FormItem>
          );
        }}
      />
    </div>
  );
}
