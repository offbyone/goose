import { useState, useEffect } from 'react';
import { getTools } from '../../api';

export const useToolCount = (triggerRefetch?: unknown) => {
  const [toolCount, setToolCount] = useState<number | null>(null);

  useEffect(() => {
    let retryCount = 0;
    const maxRetries = 5;
    const initialDelay = 1000;
    let mounted = true;

    const fetchTools = async () => {
      try {
        const response = await getTools();
        // Hack to get around getTools returning an empty list if the engine is not ready yet
        if (!response.error && response.data) {
          // If we get a valid response with no tools, update immediately
          if (response.data.length === 0) {
            if (retryCount < maxRetries) {
              // Only retry if we haven't hit max retries
              retryCount++;
              const delay = initialDelay * Math.pow(2, retryCount - 1);
              console.log(
                `Got empty tool list, retrying (${retryCount}/${maxRetries}) in ${delay}ms...`
              );
              // Set the current count to 0
              if (mounted) {
                setToolCount(0);
              }
              setTimeout(fetchTools, delay);
            } else {
              // Max retries reached, confirm zero tools
              if (mounted) {
                setToolCount(0);
              }
            }
          } else {
            // We got tools, update the count
            console.log(`Got tool count: ${response.data.length} (after ${retryCount} retries)`);
            if (mounted) {
              setToolCount(response.data.length);
            }
          }
        } else {
          if (mounted) {
            setToolCount(0);
          }
        }
      } catch (err) {
        console.error('Error fetching tools:', err);
        if (mounted) {
          setToolCount(0);
        }
      }
    };

    fetchTools();

    return () => {
      mounted = false;
    };
  }, [triggerRefetch]);

  return toolCount;
};
