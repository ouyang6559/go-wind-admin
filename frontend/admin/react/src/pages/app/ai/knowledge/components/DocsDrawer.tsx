import { useEffect, useState } from 'react';
import { Button, Empty, Input, Modal, Popconfirm, Table, Tag, App } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { DeleteOutlined, UploadOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useQueryClient } from '@tanstack/react-query';
import type { aiservicev1_AiDoc as AiDoc, aiservicev1_AiKnowledgeBase as AiKnowledgeBase } from '@/api/generated/admin/service/v1';
import { fetchListAiDocs, useDeleteAiDoc, useUploadAiDoc } from '@/api/hooks/ai-knowledge';

interface DocsDrawerProps {
  open: boolean;
  data?: AiKnowledgeBase;
  onClose: () => void;
}

export default function DocsDrawer({ open, data, onClose }: DocsDrawerProps) {
  const { t } = useTranslation('aiKnowledge');
  const { message } = App.useApp();
  const queryClient = useQueryClient();

  const [docs, setDocs] = useState<AiDoc[]>([]);
  const [loading, setLoading] = useState(false);
  const [name, setName] = useState('');
  const [content, setContent] = useState('');

  const uploadMutation = useUploadAiDoc();
  const deleteMutation = useDeleteAiDoc();

  const loadDocs = (baseId: number) => {
    setLoading(true);
    fetchListAiDocs(baseId)
      .then((res) => setDocs((res.items ?? []) as AiDoc[]))
      .catch((error: Error) => {
        console.error('fetch ai docs failed:', error);
        message.error(t('fetchFailed'));
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    if (open && data?.id) {
      loadDocs(data.id);
    }
    if (!open) {
      setName('');
      setContent('');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, data?.id]);

  const handleUpload = async () => {
    if (!data?.id || !name.trim() || !content.trim()) return;
    try {
      const res = await uploadMutation.mutateAsync({ baseId: data.id, name: name.trim(), content });
      message.success(t('uploadSuccess', { chunks: res.chunkCount ?? 0 }));
      setName('');
      setContent('');
      loadDocs(data.id);
      queryClient.invalidateQueries({ queryKey: ['listAiKnowledgeBases'] });
    } catch (error) {
      console.error('upload ai doc failed:', error);
      message.error((error as Error).message || t('uploadFailed'));
    }
  };

  const handleDelete = async (doc: AiDoc) => {
    if (!data?.id) return;
    try {
      await deleteMutation.mutateAsync({ baseId: data.id, id: doc.id! });
      message.success(t('deleteSuccess'));
      loadDocs(data.id);
      queryClient.invalidateQueries({ queryKey: ['listAiKnowledgeBases'] });
    } catch (error) {
      console.error('delete ai doc failed:', error);
      message.error((error as Error).message || t('fetchFailed'));
    }
  };

  const columns: ColumnsType<AiDoc> = [
    { title: t('docName'), dataIndex: 'name', ellipsis: true },
    {
      title: t('docStatus'),
      dataIndex: 'status',
      width: 90,
      render: (_, record) =>
        record.status === 'READY' ? (
          <Tag color="success">READY</Tag>
        ) : (
          <Tag color="error">{record.status || 'FAILED'}</Tag>
        ),
    },
    { title: t('docChunks'), dataIndex: 'chunkCount', width: 80 },
    {
      title: t('action'),
      width: 80,
      render: (_, record) => (
        <Popconfirm title={t('deleteDocConfirm')} onConfirm={() => handleDelete(record)}>
          <Button type="link" size="small" danger icon={<DeleteOutlined />} />
        </Popconfirm>
      ),
    },
  ];

  return (
    <Modal
      title={`${t('docsTitle', { name: data?.name ?? '' })}`}
      open={open}
      onCancel={onClose}
      footer={null}
      width={680}
      destroyOnHidden
    >
      {/* 上传区：纯文本直接入库（切片 → 向量化） */}
      <div className="mb-4 space-y-2">
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t('docNamePlaceholder')}
          maxLength={100}
        />
        <Input.TextArea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder={t('docContentPlaceholder')}
          rows={5}
          maxLength={50000}
          showCount
        />
        <Button
          type="primary"
          icon={<UploadOutlined />}
          loading={uploadMutation.isPending}
          disabled={!name.trim() || !content.trim()}
          onClick={handleUpload}
        >
          {t('upload')}
        </Button>
      </div>

      <Table<AiDoc>
        rowKey="id"
        size="small"
        loading={loading}
        columns={columns}
        dataSource={docs}
        pagination={false}
        locale={{ emptyText: <Empty description={t('noDocs')} image={Empty.PRESENTED_IMAGE_SIMPLE} /> }}
      />
    </Modal>
  );
}
