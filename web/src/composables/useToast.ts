import { ref } from 'vue'

export function useToast() {
  const toastMessage = ref('')
  const toastVisible = ref(false)
  let toastTimeout: ReturnType<typeof setTimeout> | null = null

  function showToast(message: string) {
    toastMessage.value = message
    toastVisible.value = true
    if (toastTimeout)
      clearTimeout(toastTimeout)
    toastTimeout = setTimeout(() => {
      toastVisible.value = false
    }, 4000)
  }

  return {
    toastMessage,
    toastVisible,
    showToast,
  }
}
